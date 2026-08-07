#[derive(Clone, Debug)]
pub enum GitEvent {
    BranchChanged { branch: String },
    StatusChanged { path: String },
}
