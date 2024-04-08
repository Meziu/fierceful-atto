//! Module including definitions for [`Team`], groups of [`Member`]s that fight in a [`Battle`](crate::battle::Battle).

use crate::member::Member;

/// Coalition made up of multiple fighting [`Member`]s.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Team {
    name: String,
    member_list: Vec<Member>,
}

impl Team {
    /// Create a new [`Team`] object using a list of members associated to it.
    pub fn new(name: String, member_list: Vec<Member>) -> Self {
        Self { name, member_list }
    }

    /// Returns this team's name.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns a reference to the internal member list.
    pub fn member_list(&self) -> &[Member] {
        &self.member_list
    }

    /// Returns a mutable reference to the internal member list.
    pub fn member_list_mut(&mut self) -> &mut [Member] {
        &mut self.member_list
    }

    /// Returns a reference to one associated member.
    pub fn member(&self, member_id: usize) -> Option<&Member> {
        self.member_list.get(member_id)
    }

    /// Returns a mutable reference to one associated member.
    pub fn member_mut(&mut self, member_id: usize) -> Option<&mut Member> {
        self.member_list.get_mut(member_id)
    }
}
