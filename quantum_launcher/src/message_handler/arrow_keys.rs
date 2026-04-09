use iced::{Task, widget::scrollable};
use ql_core::Instance;

use crate::{
    config::sidebar::{SidebarNode, SidebarNodeKind},
    state::{Launcher, Message, State},
};

struct SidebarWalker<'a> {
    selected_instance: &'a mut Option<Instance>,

    shift_pressed: bool,
    going_down: bool,

    is_the_one: bool,
    prev_search_idx: usize,
    search_idx: usize,
    total_len: usize,
    has_been_found: bool,
}

impl<'a> SidebarWalker<'a> {
    fn new(
        selected_instance: &'a mut Option<Instance>,
        shift_pressed: bool,
        going_down: bool,
    ) -> Self {
        Self {
            selected_instance,
            shift_pressed,
            going_down,
            is_the_one: false,
            prev_search_idx: 0,
            search_idx: 0,
            total_len: 0,
            has_been_found: false,
        }
    }

    fn walk(&mut self, nodes: &mut [SidebarNode], (): ()) -> bool {
        self.has_been_found = false;
        if nodes.len() == 0 {
            return false;
        }

        self.reset();
        self.try_walk(nodes);

        if !self.has_been_found && !self.shift_pressed {
            // Try again, forcing shift press
            self.shift_pressed = true;
            self.reset();
            self.try_walk(nodes);
        }

        if !self.going_down {
            self.search_idx = self.total_len - self.search_idx - 1;
            self.prev_search_idx = self.total_len - self.prev_search_idx - 1;
        }

        if self.has_been_found {
        } else {
            self.search_idx = self.prev_search_idx;
        }
        self.has_been_found
    }

    fn reset(&mut self) {
        self.is_the_one = false;
        self.has_been_found = false;
        self.search_idx = 0;
        self.prev_search_idx = 0;
        self.total_len = 0;
    }

    fn try_walk(&mut self, nodes: &mut [SidebarNode]) {
        if self.going_down {
            for node in nodes {
                self.walk_node(node);
            }
        } else {
            for node in nodes.iter_mut().rev() {
                self.walk_node(node);
            }
        }
    }

    fn walk_node(&mut self, node: &mut SidebarNode) {
        match &mut node.kind {
            SidebarNodeKind::Folder(folder) => {
                // folder/
                // - elem1
                // - elem2

                if self.going_down {
                    // Here folder is displayed above elements,
                    // so if moving down, gotta count the folder first
                    // before its elements
                    self.total_len += 1;
                }
                if !self.is_the_one || folder.is_expanded || self.shift_pressed {
                    let found_old = self.has_been_found;

                    let old_len = self.total_len;
                    self.try_walk(&mut folder.children);

                    if found_old != self.has_been_found {
                        // Found from this folder!
                        folder.is_expanded = true;
                    } else if !folder.is_expanded {
                        // Don't count elements inside a closed folder
                        self.total_len = old_len;
                    }
                }
                if !self.going_down {
                    // If moving up, folder comes last after it's elements
                    self.total_len += 1;
                }
            }
            SidebarNodeKind::Instance(kind) => {
                if self.has_been_found {
                    // Do nothing, we already found the one we want,
                    // just counting total length
                } else if self.is_the_one {
                    *self.selected_instance = Some(Instance {
                        name: node.name.clone(),
                        kind: *kind,
                    });
                    self.search_idx = self.total_len;
                    self.has_been_found = true;
                } else if let Some(instance) = self.selected_instance {
                    if node.name == instance.name && *kind == instance.kind {
                        self.prev_search_idx = self.total_len;
                        self.is_the_one = true;
                    }
                } else {
                    // If no instance selected, pick the first one
                    self.prev_search_idx = self.total_len;
                    self.is_the_one = true;
                }
                self.total_len += 1;
            }
        }
    }
}

impl Launcher {
    pub(super) fn key_change_selected_instance(&mut self, going_down: bool) -> Task<Message> {
        let Some(sidebar) = &mut self.config.sidebar else {
            return Task::none();
        };
        let sidebar_height = {
            let State::Launch(menu) = &self.state else {
                return Task::none();
            };
            menu.sidebar_scroll.remaining
        };

        let mut walker = SidebarWalker::new(
            &mut self.selected_instance,
            self.modifiers_pressed.shift(),
            going_down,
        );

        let did_scroll = walker.walk(&mut sidebar.list, ());

        if did_scroll {
            let scroll_pos = (walker.search_idx as f32 - 1.0) / (walker.total_len as f32 - 1.0);
            let scroll_pos = scroll_pos.max(0.0) * sidebar_height;

            let scroll_task = scrollable::scroll_to(
                scrollable::Id::new("MenuLaunch:sidebar"),
                scrollable::AbsoluteOffset {
                    x: 0.0,
                    y: scroll_pos,
                },
            );

            Task::batch([scroll_task, self.on_selecting_instance()])
        } else {
            Task::none()
        }
    }
}
