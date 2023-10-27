use super::file::Project;

impl Project {
    fn new(i: Project) -> Self {
        Self { ..i }
    }

    fn validate(&self) -> bool {
        let Project {
            project_id,
            associated_projects,
            dataset_prepare,
            filplus_guideline,
            dataset_life_span,
            geographis,
            retrival_frequency,
            previous_stoarge,
            public_dataset,
            providers,
            data_sample_link,
            stored_data_desc,
            distribution,
            history,
        } = self;
        project_id.len() > 0
            && associated_projects.len() > 0
            && dataset_prepare.len() > 0
            && filplus_guideline.len() > 0
            && dataset_life_span.len() > 0
            && geographis.len() > 0
            && retrival_frequency.len() > 0
            && previous_stoarge.len() > 0
            && public_dataset.len() > 0
            && providers.len() > 0
            && data_sample_link.len() > 0
            && stored_data_desc.len() > 0
            && distribution.len() > 0
            && history.len() > 0
    }
}
