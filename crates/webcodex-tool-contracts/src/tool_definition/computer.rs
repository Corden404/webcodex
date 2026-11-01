use super::RunnerCapabilityRequirement::FileWrite;
use super::ToolVisibility::ModelVisible;
use super::{
    def, model_spec, permission_risk, require_all_scopes, require_any_scopes, ToolDefinition,
    PERMISSION_RISK_WRITE, TOOL_CATEGORY_COMPUTER,
};
use crate::metadata::{
    ToolPathHint::{Artifact, None as NoPath},
    ToolRisk::{ComputerControl as ComputerControlRisk, ProjectWrite, Read},
    COMPUTER_CONTROL, COMPUTER_LAUNCH, COMPUTER_READ, PROJECT_WRITE, TOOL_PROVIDER_CONTROL,
};

const COMPUTER_CONTROL_GATEWAY_SCOPES: &[&str] = &[COMPUTER_CONTROL, COMPUTER_LAUNCH];

pub(super) const DEFINITIONS: &[ToolDefinition] = &[
    model_spec(
        def(
            "observe_computer",
            super::ToolAuditPolicy::typed_semantic(
                super::ToolAuditSemanticResultPolicy::ComputerObservation,
            ),
            ModelVisible, TOOL_CATEGORY_COMPUTER, None, TOOL_PROVIDER_CONTROL,
            super::ToolSemanticContract { effect: super::ToolEffect::Observe, risk: Read, approval: super::ToolApprovalPolicy::None, idempotency: super::ToolIdempotency::PureRead },
            Some(COMPUTER_READ), false, NoPath, false, false, super::ToolSessionEvidencePolicy::NONE,
        ),
        "Read-only Computer gateway: targets/windows/displays/applications, accessibility_status/accessibility_tree/find_elements/element_state, snapshot_window/snapshot_display, read_clipboard. For action eligibility, use element_state with exact client_id/surface_id/element_id: can_press/can_focus/can_input_text/can_select guide controls, never grant authority; control revalidates. Role/value alone is insufficient. Requery only when changed state matters. element_state preserves observation_generation; stale IDs need fresh tree/search. Tree/search replaces all IDs on that surface, including root_element_id. For truncated searches, choose a branch in a fresh tree and query its latest root_element_id. Scoped reads need an updated Runner and verified scope receipt. Limits: depth 8, nodes 256, lineage 32; native deadlines apply. Truncated zero results are inconclusive; unseen siblings beyond the node cap are not paginated. Windows scroll reports horizontal_percent/vertical_percent with computer_scroll: null axis=non-scrollable, null object=unsupported/protected, omitted=unknown. Positions grant no authority. Windows ComboBox values grant no focus/input. Exact scopes, capabilities, protected-content, image/clipboard bounds and snapshot-generation guards apply. No activation, launch, control, pointer/clipboard writes, project artifact save, shell fallback or uncertain-effect retry.",
    ),
    require_any_scopes(
        permission_risk(
            model_spec(
                def(
                    "control_computer",
                    super::ToolAuditPolicy::typed_semantic(
                        super::ToolAuditSemanticResultPolicy::ComputerControl,
                    ),
                    ModelVisible, TOOL_CATEGORY_COMPUTER, None, TOOL_PROVIDER_CONTROL,
                    super::ToolSemanticContract { effect: super::ToolEffect::Execute, risk: ComputerControlRisk, approval: super::ToolApprovalPolicy::Standard, idempotency: super::ToolIdempotency::NonIdempotent },
                    None, false, NoPath, true, false, super::ToolSessionEvidencePolicy::NONE,
                ),
                "Effectful Computer gateway; exact schema action/target. element_state can_press/can_focus/can_input_text/can_select and scroll guide eligibility, not authority. Windows press needs InvokePattern; focus/input need foreground AXTextField; input needs exact focus, writable empty value. select uses SelectionItemPattern on ListItem/TabItem, replaces selection; requires computer_selection+computer_control. scroll uses exact ScrollPattern container, up/down/left/right, small/large (omitted/null=small), at most one native increment; requires computer_scroll+computer_control. position_before/position_after may be unchanged/intermediate; re-observe content. key uses schema vocabulary; computer_key_input plus computer_key_chords for extra keys, or computer_letter_shortcuts for Ctrl+J/Ctrl+P. macOS navigation only. Keys do not promise text or app commands. Shift+Tab: key=tab,modifiers=[shift]. Windows option=Alt; command/option+Tab/option+Escape/control+Escape/control+option+Delete rejected. Keys need foreground/focused surface, unprotected focus and released held keys. Only press/input_text/scroll accept observe_after={action:accessibility_tree,max_depth?,max_nodes?}: preflight computer:read and read/action capabilities; pin Runner/window. output.post_observation={success,output,error?}: fresh, not UI-settled; tree replaces IDs. Read failure preserves action success; failed/uncertain action skips read. Scope, permission, identity/snapshot guards apply; no implicit focus/activation, scripts or uncertain-effect replay.",
            ),
            PERMISSION_RISK_WRITE,
        ),
        COMPUTER_CONTROL_GATEWAY_SCOPES,
    ),
    require_all_scopes(
        model_spec(
            def(
                "save_computer_snapshot",
                super::ToolAuditPolicy::typed_fields(&[
                    super::ToolAuditResultField::value("project"),
                    super::ToolAuditResultField::value("path"),
                    super::ToolAuditResultField::value("client_id"),
                    super::ToolAuditResultField::value("surface_id"),
                    super::ToolAuditResultField::value("source_width"),
                    super::ToolAuditResultField::value("source_height"),
                    super::ToolAuditResultField::presence("region_present", "region"),
                    super::ToolAuditResultField::value("width"),
                    super::ToolAuditResultField::value("height"),
                    super::ToolAuditResultField::value("mime_type"),
                    super::ToolAuditResultField::value("file_bytes"),
                    super::ToolAuditResultField::value("saved"),
                ]),
                ModelVisible,
                TOOL_CATEGORY_COMPUTER,
                Some(FileWrite),
                TOOL_PROVIDER_CONTROL,
                super::ToolSemanticContract {
                    effect: super::ToolEffect::Mutate,
                    risk: ProjectWrite,
                    approval: super::ToolApprovalPolicy::Standard,
                    idempotency: super::ToolIdempotency::NonIdempotent,
                },
                Some(PROJECT_WRITE),
                true,
                Artifact,
                false,
                false,
                super::ToolSessionEvidencePolicy::NONE,
            ),
            "Save one exact window snapshot as a create-only project artifact without returning image bytes. Reuses observe_computer(action=snapshot_window) region/downscale semantics and requires computer:read plus project:write. No overwrite or encoding control. Unknown writes require artifact-metadata reconciliation before retry.",
        ),
        &[PROJECT_WRITE, COMPUTER_READ],
    ),
];
