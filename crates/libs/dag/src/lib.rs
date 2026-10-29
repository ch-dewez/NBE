use std::collections::HashMap;

use thiserror::Error;

pub type DagNodeId = usize;

pub struct DagNode<T> {
    pub value: T,

    inputs: Vec<DagNodeId>,
    outputs: Vec<DagNodeId>,
}

pub struct DAG<T> {
    nodes: HashMap<DagNodeId, DagNode<T>>,
    starting_nodes: Vec<DagNodeId>, // No inputs
    ending_nodes: Vec<DagNodeId>,   // No outputs
    next_id: DagNodeId,
}

impl<T> Default for DAG<T> {
    fn default() -> Self {
        Self {
            nodes: HashMap::new(),
            starting_nodes: Vec::new(),
            ending_nodes: Vec::new(),
            next_id: 0,
        }
    }
}

#[derive(Debug, Error)]
#[error("The requested node of the Directed Acyclic Graph doesn't exist (id={0})")]
pub struct DagNodeNotFoundErr(DagNodeId);

impl<T> DAG<T> {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_node_from_inputs(
        &mut self,
        inputs: Vec<DagNodeId>,
        value: T,
    ) -> Result<DagNodeId, DagNodeNotFoundErr> {
        for id in inputs.iter() {
            let input = self.nodes.get_mut(id).ok_or(DagNodeNotFoundErr(*id))?;
            input.outputs.push(*id);
            if let Some(index) = self
                .ending_nodes
                .iter()
                .find(|filter_id| **filter_id == *id)
            {
                self.ending_nodes.remove(*index);
            }
        }

        let id = self.next_id;
        self.next_id += 1;

        if inputs.is_empty() {
            self.starting_nodes.push(id);
        }
        self.ending_nodes.push(id);

        let node = DagNode {
            value,
            inputs,
            outputs: Vec::new(),
        };

        self.nodes.insert(id, node);
        Ok(id)
    }

    pub fn add_node_from_outputs(
        &mut self,
        outputs: Vec<DagNodeId>,
        value: T,
    ) -> Result<DagNodeId, DagNodeNotFoundErr> {
        for id in outputs.iter() {
            let node = self.nodes.get_mut(id).ok_or(DagNodeNotFoundErr(*id))?;
            node.inputs.push(*id);
            if let Some(index) = self
                .starting_nodes
                .iter()
                .find(|filter_id| **filter_id == *id)
            {
                self.ending_nodes.remove(*index);
            }
        }

        let id = self.next_id;
        self.next_id += 1;

        if outputs.is_empty() {
            self.ending_nodes.push(id);
        }
        self.starting_nodes.push(id);

        let node = DagNode {
            value,
            inputs: Vec::new(),
            outputs,
        };

        self.nodes.insert(id, node);
        Ok(id)
    }

    pub fn get_node(&self, node: DagNodeId) -> Option<&DagNode<T>> {
        self.nodes.get(&node)
    }

    pub fn get_node_mut(&mut self, node: DagNodeId) -> Option<&mut DagNode<T>> {
        self.nodes.get_mut(&node)
    }

    pub fn get_starting_nodes(&self) -> &[DagNodeId] {
        &self.starting_nodes
    }

    pub fn get_ending_nodes(&self) -> &[DagNodeId] {
        &self.ending_nodes
    }

    pub fn get_all_nodes(&self) -> &HashMap<DagNodeId, DagNode<T>> {
        &self.nodes
    }

    pub fn get_all_nodes_mut(&mut self) -> &mut HashMap<DagNodeId, DagNode<T>> {
        &mut self.nodes
    }
}
