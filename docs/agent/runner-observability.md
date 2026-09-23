# Runner observability

Issue [#579](https://github.com/yyjeqhc/webcodex/issues/579) converges public Runner
observations on one vocabulary. Implementation baseline:
`5e6abf79b651c27d142299813a16a54d6ecadb73`.

| Previous observation | Canonical observation |
| --- | --- |
| `runtime_status.agents` | `runtime_status.runners` |
| `agents.clients` and `agents.summary.clients` | `runners.clients` |
| `list_runners.clients` and `list_runners.runners` | `list_runners.runners` |
| `agent_instance_id` | `runner_instance_id` |
| `agent_protocol_generation` | `runner_protocol_generation` |
| `mismatched_agents_count`, `source_mismatched_agents_count` | `mismatched_runners_count`, `source_mismatched_runners_count` |
| Admin `agents_total`, `agents_online` | `runners_total`, `runners_online` |
| CLI summary/server status `agents` | `runners` |

Full, compact, summary, and focused status retain their existing selection and
count semantics. Summary contains aggregates only. Compact clients retain health
facts formerly held by the duplicate health list: last-seen age, enabled project
count, inventory state, pending requests, active Jobs, and concurrency. Full-only
owner, policy, and capability details remain outside compact output. Focused
status describes only the selected Runner and includes its inventory state.

Admin, Console, CLI JSON/text, pairing owner checks, provider discovery, coding
startup health checks, and platform readiness readers consume this same tree.
An online peer does not make an unavailable owning Runner healthy. Project and
Job ownership, stale-instance fences, authorization, and readiness rules remain
authoritative. Runtime status uses `projects.runner_registered`; the separate
`list_projects.source` classification retains its existing `agent_registered` value.

Registration and transport DTOs, `/api/agents/*`, build-info
`agent_protocol_generation`, credential names, persisted records, diagnostic
trace fields, reason codes, canonical `agent:<client_id>:<project_id>` IDs, Coding
Agents, and durable Agent APIs retain their established contracts. This is not
a Runner wire-generation migration and does not add permanent observation aliases.

## Upgrade status

The management contract stays `[1, 1]` and the Runner wire generation stays
unchanged while the coordinated upgrade strategy is awaiting upstream
confirmation. This is an unresolved integration/release condition, not evidence
that old and new observation clients interoperate.

Older CLI versions read `/agents/clients` or `/agents/summary/clients`; against
the new Server they may report an empty fleet or an unavailable Runner. Desktop
indirectly consumes these observations through CLI commands. The existing
management-generation check cannot detect this schema difference.

The candidate implementation updates Server, CLI, Console/Admin, and scripts
together and must be reviewed as one change. Supported mixed Desktop/CLI/Server
combinations and the deployment order remain to be agreed upstream before
publication. Do not publish the intermediate Server-only commits independently.

See [testing guidance](../TESTING.md#runner-observability-contract) for behavior
tests and the CI guard. Native Windows readiness tests and Linux socket-activation
E2E are separate evidence; passing one does not establish the other.
