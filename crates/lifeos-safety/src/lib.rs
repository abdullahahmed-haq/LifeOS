use lifeos_domain::{ActorKind, PermissionDecision, PermissionEvaluation, PermissionPolicy};

/// Evaluates a bounded policy set. Actor identity is resolved by the caller's
/// trusted ingress; this module only maps that identity and an operation to a
/// deterministic decision.
pub fn evaluate(
    actor: ActorKind,
    operation: &str,
    policies: &[PermissionPolicy],
) -> PermissionEvaluation {
    let matched = policies.iter().find(|policy| {
        policy.enabled && policy.subject_kind == actor && policy.operation == operation
    });
    let decision = matched.map_or_else(
        || default_decision(&actor),
        |policy| policy.decision.clone(),
    );
    PermissionEvaluation {
        actor,
        operation: operation.to_owned(),
        decision,
        matched_policy_id: matched.map(|policy| policy.id.clone()),
    }
}

fn default_decision(actor: &ActorKind) -> PermissionDecision {
    match actor {
        ActorKind::User => PermissionDecision::Allow,
        ActorKind::Ai | ActorKind::Mcp | ActorKind::Automation | ActorKind::Obsidian => {
            PermissionDecision::Ask
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn policy(decision: PermissionDecision) -> PermissionPolicy {
        PermissionPolicy {
            id: "policy-1".into(),
            subject_kind: ActorKind::Ai,
            operation: "area.create".into(),
            decision,
            enabled: true,
            revision: 1,
        }
    }

    #[test]
    fn local_user_is_allowed_and_external_actors_ask_by_default() {
        assert_eq!(
            evaluate(ActorKind::User, "area.create", &[]).decision,
            PermissionDecision::Allow
        );
        assert_eq!(
            evaluate(ActorKind::Ai, "area.create", &[]).decision,
            PermissionDecision::Ask
        );
    }

    #[test]
    fn exact_enabled_policy_overrides_the_default() {
        let policy = policy(PermissionDecision::Deny);
        let evaluation = evaluate(ActorKind::Ai, "area.create", &[policy]);
        assert_eq!(evaluation.decision, PermissionDecision::Deny);
        assert_eq!(evaluation.matched_policy_id.as_deref(), Some("policy-1"));
    }
}
