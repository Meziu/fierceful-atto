//! Definitions for [`Team`], groups of [`Member`]s that fight in a [`Battle`](crate::battle::Battle).

use std::marker::PhantomData;

use crate::member::{Member, Properties, Statistics};

/// Coalition made up of multiple fighting [`Member`]s.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Team<M: Member<S, P>, S: Statistics, P: Properties> {
    name: String,
    member_list: Vec<M>,
    member_types: PhantomData<(S, P)>,
}

impl<M: Member<S, P>, S: Statistics, P: Properties> Team<M, S, P> {
    /// Create a new [`Team`] object using a list of members associated to it.
    pub fn new(name: String, member_list: Vec<M>) -> Self {
        log::debug!(
            "Team \"{name}\" was created with {} member(s)",
            member_list.len()
        );

        Self {
            name,
            member_list,
            member_types: PhantomData,
        }
    }

    /// Returns this team's name.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns a reference to the internal member list.
    pub fn member_list(&self) -> &[M] {
        &self.member_list
    }

    /// Returns a mutable reference to the internal member list.
    pub fn member_list_mut(&mut self) -> &mut [M] {
        &mut self.member_list
    }

    /// Returns a reference to one associated member.
    pub fn member(&self, member_id: usize) -> Option<&M> {
        self.member_list.get(member_id)
    }

    /// Returns a mutable reference to one associated member.
    pub fn member_mut(&mut self, member_id: usize) -> Option<&mut M> {
        self.member_list.get_mut(member_id)
    }
}
