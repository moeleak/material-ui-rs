//! Android navigation hierarchy, kept independent of the platform runtime for tests.

use iced::time::Instant;
use material_ui_rs::widget::{navigation, tabs};

use super::ShowcasePage;

#[cfg(target_os = "android")]
pub(super) const TABS_ID: &str = "showcase-component-tabs";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Section {
    Components,
    Navigation,
    Structure,
}

pub(super) const DESTINATIONS: [navigation::Destination<Section>; 3] = [
    navigation::Destination::new(Section::Components, "widgets", "Components"),
    navigation::Destination::new(Section::Navigation, "navigation", "Navigation"),
    navigation::Destination::new(Section::Structure, "layers", "Structure"),
];

pub(super) const COMPONENT_PAGES: [(ShowcasePage, &str); 4] = [
    (ShowcasePage::Inputs, "Inputs"),
    (ShowcasePage::Controls, "Controls"),
    (ShowcasePage::Feedback, "Feedback"),
    (ShowcasePage::Surfaces, "Surfaces"),
];

#[derive(Debug)]
pub(super) struct Navigation {
    pub(super) sections: navigation::NavigationState<Section>,
    pub(super) component_tabs: tabs::State,
    component_page: ShowcasePage,
}

impl Default for Navigation {
    fn default() -> Self {
        Self {
            sections: navigation::NavigationState::new(Section::Components),
            component_tabs: tabs::State::new(0),
            component_page: ShowcasePage::Inputs,
        }
    }
}

impl Navigation {
    pub(super) fn page_for(&self, section: Section) -> ShowcasePage {
        match section {
            Section::Components => self.component_page,
            Section::Navigation => ShowcasePage::Navigation,
            Section::Structure => ShowcasePage::Structure,
        }
    }

    pub(super) fn select_page(
        &mut self,
        page: ShowcasePage,
        now: Instant,
        layout: navigation::AdaptiveLayout,
    ) {
        let section = match page {
            ShowcasePage::Navigation => Section::Navigation,
            ShowcasePage::Structure => Section::Structure,
            _ => {
                self.component_page = page;
                let index = COMPONENT_PAGES
                    .iter()
                    .position(|(id, _)| *id == page)
                    .unwrap();
                self.component_tabs
                    .select(index, now, tabs::Variant::Secondary);
                Section::Components
            }
        };
        self.sections.select(section, now, layout);
    }

    pub(super) fn advance(&mut self, now: Instant) {
        let _ = self.sections.advance(now);
        let _ = self.component_tabs.advance(now);
    }

    pub(super) fn is_animating(&self) -> bool {
        self.sections.is_animating() || self.component_tabs.is_animating()
    }
}
