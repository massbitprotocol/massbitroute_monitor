use crate::fisherman_service::{ComponentReport, FishermanService};
use crate::CONFIG;
use anyhow::Error;
use log::{debug, info};
use mbr_check_component::check_module::check_module::{CheckMkReport, ComponentInfo};
use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;

#[async_trait::async_trait]
pub trait CheckDataCorrectness {
    async fn check_data(
        &self,
        list_providers: Arc<RwLock<Vec<ComponentInfo>>>,
    ) -> Result<HashMap<ComponentInfo, ComponentReport>, Error>;
}

#[async_trait::async_trait]
impl CheckDataCorrectness for FishermanService {
    async fn check_data(
        &self,
        list_providers: Arc<RwLock<Vec<ComponentInfo>>>,
    ) -> Result<HashMap<ComponentInfo, ComponentReport>, Error> {
        // Copy list provider
        let mut list_providers_clone;
        {
            list_providers_clone = (*list_providers.read().await).clone();
        }

        let mut bad_components: HashMap<ComponentInfo, ComponentReport> = HashMap::new();
        let mut collect_reports: HashMap<ComponentInfo, Vec<CheckMkReport>> = HashMap::new();
        for n in 0..CONFIG.number_of_samples {
            info!("Run {} times", n + 1);
            if let Ok(reports) = self
                .check_component_service
                .check_components(&CONFIG.check_task_list_fisherman, &list_providers_clone)
                .await
            {
                debug!("reports:{:?}", reports);
                for (component, report) in reports {
                    // with each component collect reports in to vector
                    match collect_reports.entry(component) {
                        Entry::Occupied(o) => {
                            o.into_mut().push(report);
                        }
                        Entry::Vacant(v) => {
                            v.insert(vec![report]);
                        }
                    }
                }
            };

            tokio::time::sleep(Duration::from_millis(CONFIG.sample_interval_ms)).await;
        }

        debug!("collect_reports: {:?}", collect_reports);
        // Calculate average report
        for (component, reports) in collect_reports.iter() {
            info!("component:{:?}", component.id);
            bad_components.insert(component.clone(), ComponentReport::from(reports));
        }

        // Display report for debug
        for (component, report) in bad_components.iter() {
            info!("id: {}, type: {:?}, chain {:?}, request_number: {}, success_number: {}, response_time_ms:{:?}ms, unhealthy: {}",
                    component.id,
                    component.component_type,
                    component.blockchain,
                    report.request_number,
                    report.success_number,
                    report.response_time_ms,
                    report.is_unhealthy(&component.component_type)
                );
        }
        // Filter bad component only
        bad_components.retain(|component, report| report.is_unhealthy(&component.component_type));
        Ok(bad_components)
    }
}
