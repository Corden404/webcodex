# Local Computer Control

Windows Desktop shows Computer activity in its native tray. On a saved environment
with a local Runner, **Pause Computer Control** blocks further native input for all
remote clients using that Runner. **Resume Computer Control** requires an explicit
local action. The main Desktop window can be closed while the tray and Runner remain
running.

The status distinguishes observation, control requests, idle, paused control,
paused control with observation, and pausing while an admitted control request is
still in flight. Short requests leave a separate recent-activity indication for
1.5 seconds. Unavailable local state is displayed as unavailable.

Pausing covers pointer movement/clicks, keyboard and text input, window activation,
application launch, clipboard writes and semantic control actions. Observation
permissions remain unchanged, including screenshots, accessibility/window queries
and clipboard reads. A pause does not undo an operation that already entered native
execution. Resume stays disabled until admitted controls finish.

The tray also offers an optional screen-edge hint and an optional
**Ctrl+Alt+Shift+F10** pause shortcut. Both start disabled for each Desktop run.
The shortcut never resumes control. Registration conflicts are shown in the tray;
Escape remains available to the foreground application. The edge hint is placed
on up to sixteen connected displays without activation or pointer interception and
requests Windows capture exclusion. Actual capture exclusion and full-screen
appearance depend on the Windows compositor and capture provider.

## Execution and recovery

`webcodex-computer-control` owns the local gate and bounded management protocol;
`webcodex-computer` retains native target validation and Browser handoff reservations.
The Runner and its interactive helper use the same gate for request admission and
native effect entry. Resume never releases a Browser user handoff.

The additive `computer_local_control` capability declares software support even
before an interactive helper is ready; control waits for its first epoch.
The Server stamps control requests with the Runner's advertised local control
epoch. Admission, queued dispatch and native execution preserve that version.
Pause/resume, Runner replacement and helper session invalidation retire older
control requests. They are rejected rather than replayed, including clipboard
writes that do not carry a window or element handle. New control requires a fresh
request after local resume and observation of current desktop state.

The pause setting is stored independently of opaque Computer IDs. Restarting the
Runner or helper preserves a confirmed pause and creates a fresh control epoch.
Malformed saved state prevents enabling input. A failed pause persistence attempt
leaves the in-memory gate closed and reports an error; a retry must persist before
it can confirm success.

The Windows management pipe accepts only status/pause/resume, is scoped to the
interactive session, rejects remote pipe clients, and grants access to its creating
account and LocalSystem. Desktop pins the server executable and the observed epoch.
Management is independent of the serialized native-operation pipe. It carries no
credentials, screenshots, window titles, typed text or clipboard contents. There is
no remote Computer tool for resuming the gate.

Update Desktop, Runner and Server together. An updated managed Windows Runner
rejects unfenced control from an older Server while retaining the observation
contract. Older Runners retain their existing behavior and cannot provide this
local pause guarantee. The shared state machine is platform neutral; the local
management transport and Desktop controls currently require Windows.

The focused native pipe test uses disposable local state and checks server identity,
pause/resume, stale epochs, duplicate ownership and client disconnect recovery.
It does not establish real Desktop tray rendering, shortcut registration,
multi-monitor/full-screen behavior or capture exclusion acceptance.
