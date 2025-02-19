use crate::{config::get_env_var_or_default, error::LDNError};
use fplus_database::{
    database::{
        applications::get_applications_by_clients_addresses,
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

pub async fn detect_similar_applications(
    client_address: &str,
    comparable_data: &ApplicationComparableData,
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
        .unwrap_or_default()
        .into_iter()
        .chain(
            similar_stored_data_desciptions
                .unwrap_or_default()
                .into_iter(),
        )
        .chain(
            similar_project_and_stored_data_desciptions
                .unwrap_or_default()
                .into_iter(),
        )
        .chain(similar_data_set_sample.unwrap_or_default().into_iter())
        .collect();
    let unique_addresses: Vec<String> = unique_addresses.into_iter().collect();

    let applications = get_applications_by_clients_addresses(unique_addresses)
        .await
        .map_err(|e| LDNError::New(format!("Failed to get applications from database: {}", e)))?;
    Ok(())
}

fn get_similar_texts_tfidf(documents: &Vec<Document>) -> Result<Option<Vec<String>>, LDNError> {
    let mut tokenized_documents = Vec::new();
    for doc in documents {
        tokenized_documents.push(tfidf_summarizer::tokenize(&doc.text));
    }
    let df = tfidf_summarizer::document_frequency(&tokenized_documents);
    let documents_words: Vec<String> = df.keys().cloned().collect();
    let idf = tfidf_summarizer::inverse_document_frequency(&df, tokenized_documents.len());
    let mut tfidf_result = Vec::new();

    for (_, tokens) in tokenized_documents.iter().enumerate() {
        tfidf_result.push(tfidf_summarizer::tf_idf(tokens.clone(), &idf));
    }
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
    if similar_applications.is_empty() {
        return Ok(None);
    }
    Ok(Some(similar_applications))
}

fn get_similar_texts_levenshtein(
    documents: &Vec<Document>,
) -> Result<Option<Vec<String>>, LDNError> {
    let mut similar_texts = Vec::new();
    let levenshtein_threshold = get_env_var_or_default("LEVENSHTEIN_THRESHOLD")
        .parse::<usize>()
        .map_err(|e| LDNError::New(format!("Parse tfidf threshold score to f64 failed: {}", e)))?;
    for i in 1..documents.len() {
        let similarity = levenshtein(&documents[0].text, &documents[i].text);

        if similarity < levenshtein_threshold {
            similar_texts.push(documents[i].client_address.clone());
        }
    }
    if similar_texts.is_empty() {
        return Ok(None);
    }
    Ok(Some(similar_texts))
}

fn convert_to_ndarray(
    tfidf_vectors: &Vec<HashMap<String, f64>>,
    words: &Vec<String>,
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
