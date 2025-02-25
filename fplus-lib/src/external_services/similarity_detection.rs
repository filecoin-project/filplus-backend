use super::github::github_async_new;
use crate::{config::get_env_var_or_default, error::LDNError};
use fplus_database::{
    database::{
        applications::get_distinct_applications_by_clients_addresses,
        comparable_applications::get_comparable_applications,
    },
    models::comparable_applications::ApplicationComparableData,
};
use ndarray::Array1;
use std::collections::{HashMap, HashSet};
use strsim::levenshtein;

#[derive(Debug, Clone)]
pub struct Document {
    pub client_address: String,
    pub text: String,
}

type Owner = String;
type Repo = String;
type ClientAddress = String;
type Similarities = Vec<String>;
type RepoSimilarities = HashMap<(Owner, Repo), Vec<(ClientAddress, Similarities)>>;
type SortedRepoSimilarities = Vec<((Owner, Repo), Vec<(ClientAddress, Similarities)>)>;

pub async fn detect_similar_applications(
    client_address: &str,
    comparable_data: &ApplicationComparableData,
    owner: &str,
    repo: &str,
    issue_number: &u64,
) -> Result<(), LDNError> {
    let comparable_applications = get_comparable_applications().await.map_err(|e| {
        LDNError::New(format!(
            "Failed to get comparable applications from database: {}",
            e
        ))
    })?;

    let mut projects_descriptions = Vec::new();
    projects_descriptions.push(Document {
        client_address: client_address.to_string(),
        text: comparable_data.project_desc.clone(),
    });

    let mut stored_data_descriptions = Vec::new();
    stored_data_descriptions.push(Document {
        client_address: client_address.to_string(),
        text: comparable_data.stored_data_desc.clone(),
    });

    let mut projects_and_stored_data_descriptions = Vec::new();
    projects_and_stored_data_descriptions.push(Document {
        client_address: client_address.to_string(),
        text: comparable_data.project_desc.clone() + &comparable_data.stored_data_desc.clone(),
    });

    let mut data_set_samples = Vec::new();
    data_set_samples.push(Document {
        client_address: client_address.to_string(),
        text: comparable_data.data_set_sample.clone(),
    });

    let mut existing_data_owner_name = Vec::new();
    for app in comparable_applications.iter() {
        projects_descriptions.push(Document {
            client_address: app.client_address.clone(),
            text: app.application.project_desc.clone(),
        });
        stored_data_descriptions.push(Document {
            client_address: app.client_address.clone(),
            text: app.application.stored_data_desc.clone(),
        });
        projects_and_stored_data_descriptions.push(Document {
            client_address: app.client_address.clone(),
            text: app.application.project_desc.clone() + &app.application.stored_data_desc.clone(),
        });
        data_set_samples.push(Document {
            client_address: app.client_address.clone(),
            text: app.application.data_set_sample.clone(),
        });
        if comparable_data.data_owner_name == app.application.data_owner_name {
            existing_data_owner_name.push(app.client_address.clone());
        }
    }
    let similar_project_desciptions = get_similar_texts_tfidf(&projects_descriptions)?;
    let similar_stored_data_desciptions = get_similar_texts_tfidf(&stored_data_descriptions)?;
    let similar_project_and_stored_data_desciptions =
        get_similar_texts_tfidf(&projects_and_stored_data_descriptions)?;
    let similar_data_set_sample = get_similar_texts_levenshtein(&data_set_samples)?;

    let unique_addresses: HashSet<String> = similar_project_desciptions
        .clone()
        .into_iter()
        .chain(similar_stored_data_desciptions.clone().into_iter())
        .chain(
            similar_project_and_stored_data_desciptions
                .clone()
                .into_iter(),
        )
        .chain(similar_data_set_sample.clone().into_iter())
        .chain(existing_data_owner_name.clone().into_iter())
        .collect();

    let unique_addresses: Vec<String> = unique_addresses.into_iter().collect();
    let gh = github_async_new(owner.to_string(), repo.to_string()).await?;

    if unique_addresses.is_empty() {
        let comment = "## Similarity Report\n\nNo similar applications found for the issue";
        gh.add_comment_to_issue(*issue_number, comment)
            .await
            .map_err(|e| LDNError::New(format!("Failed to get add comment to the issue: {}", e)))?;
        return Ok(());
    }

    let applications = get_distinct_applications_by_clients_addresses(unique_addresses)
        .await
        .map_err(|e| LDNError::New(format!("Failed to get applications from database: {}", e)))?;

    let mut repo_similarities: RepoSimilarities = HashMap::new();

    for application in applications {
        let repo_key = (application.owner.clone(), application.repo.clone());
        let issue_link = format!(
            "https://github.com/{}/{}/issues/{}",
            application.owner, application.repo, application.issue_number
        );

        let entry = repo_similarities.entry(repo_key).or_default();
        let mut similarities = Vec::new();

        if similar_project_and_stored_data_desciptions.contains(&application.id) {
            similarities.push("Similar project and stored data description".to_string());
        } else if similar_project_desciptions.contains(&application.id) {
            similarities.push("Similar project description".to_string());
        } else if similar_stored_data_desciptions.contains(&application.id) {
            similarities.push("Similar stored data description".to_string());
        }
        if similar_data_set_sample.contains(&application.id) {
            similarities.push("Similar data set sample".to_string());
        }
        if existing_data_owner_name.contains(&application.id) {
            similarities.push("The same data owner name".to_string());
        }

        if !similarities.is_empty() {
            entry.push((issue_link, similarities));
        }
    }

    let mut sorted_results: SortedRepoSimilarities = repo_similarities.into_iter().collect();
    sorted_results.sort_by(|owner_repo, similarities| {
        similarities
            .1
            .iter()
            .map(|(_, sim)| sim.len())
            .sum::<usize>()
            .cmp(&owner_repo.1.iter().map(|(_, sim)| sim.len()).sum::<usize>())
    });

    let comment = format!(
        "## Similarity Report\n\nThis application is similar to the following applications:\n\n{}",
        format_comment(&sorted_results)
    );
    gh.add_comment_to_issue(*issue_number, &comment)
        .await
        .map_err(|e| LDNError::New(format!("Failed to get add comment to the issue: {}", e)))?;
    Ok(())
}

fn get_similar_texts_tfidf(documents: &[Document]) -> Result<Vec<String>, LDNError> {
    let tokenized_documents: Vec<Vec<String>> = documents
        .iter()
        .map(|doc| tfidf_summarizer::tokenize(&doc.text))
        .collect();

    let df = tfidf_summarizer::document_frequency(&tokenized_documents);
    let documents_words: Vec<String> = df.keys().cloned().collect();
    let idf = tfidf_summarizer::inverse_document_frequency(&df, tokenized_documents.len());
    let tfidf_result: Vec<HashMap<String, f64>> = tokenized_documents
        .iter()
        .map(|tokens| tfidf_summarizer::tf_idf(tokens.clone(), &idf))
        .collect();

    let documents_converted_to_array = convert_to_ndarray(&tfidf_result, &documents_words);
    let mut similar_applications: Vec<String> = Vec::new();
    let tfidf_threshold = get_env_var_or_default("TFIDF_THRESHOLD")
        .parse::<f64>()
        .map_err(|e| LDNError::New(format!("Parse tfidf threshold score to f64 failed: {}", e)))?;
    for i in 1..documents_converted_to_array.len() {
        let similarity = cosine_similarity(
            &documents_converted_to_array[0],
            &documents_converted_to_array[i],
        );
        if similarity > tfidf_threshold {
            similar_applications.push(documents[i].client_address.clone());
        }
    }

    Ok(similar_applications)
}

fn get_similar_texts_levenshtein(documents: &[Document]) -> Result<Vec<String>, LDNError> {
    let levenshtein_threshold = get_env_var_or_default("LEVENSHTEIN_THRESHOLD")
        .parse::<usize>()
        .map_err(|e| {
            LDNError::New(format!(
                "Parse tfidf threshold score to usize failed: {}",
                e
            ))
        })?;

    let similar_texts: Vec<String> = documents
        .iter()
        .skip(1)
        .filter(|doc| levenshtein(&documents[0].text, &doc.text) < levenshtein_threshold)
        .map(|doc| doc.client_address.clone())
        .collect();

    Ok(similar_texts)
}

fn convert_to_ndarray(
    tfidf_vectors: &[HashMap<String, f64>],
    words: &[String],
) -> Vec<Array1<f64>> {
    tfidf_vectors
        .iter()
        .map(|doc_vector| {
            let vec: Vec<f64> = words
                .iter()
                .map(|word| *doc_vector.get(word).unwrap_or(&0.0))
                .collect();
            Array1::from(vec)
        })
        .collect()
}

fn cosine_similarity(v1: &Array1<f64>, v2: &Array1<f64>) -> f64 {
    let dot_product = v1.dot(v2);
    let norm_v1 = v1.dot(v1).sqrt();
    let norm_v2 = v2.dot(v2).sqrt();

    if norm_v1 == 0.0 || norm_v2 == 0.0 {
        0.0
    } else {
        dot_product / (norm_v1 * norm_v2)
    }
}

fn format_comment(repos: &SortedRepoSimilarities) -> String {
    repos
        .iter()
        .map(|((owner, repo), issues)| {
            format!(
                "### {}/{}\n\n{}",
                owner,
                repo,
                issues
                    .iter()
                    .map(|(issue, similarities)| {
                        format!("* {}:\n    * {}", issue, similarities.join("\n    * "))
                    })
                    .collect::<Vec<String>>()
                    .join("\n\n")
            )
        })
        .collect::<Vec<String>>()
        .join("\n\n")
}
