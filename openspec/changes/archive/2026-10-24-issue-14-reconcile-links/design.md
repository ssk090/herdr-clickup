# Design: live panes are the source of truth

## Decisions

- **One list, one prune.** `herdr pane list` returns every live pane. `store::prune` removes the links
  whose pane id is not in that set, under the state lock.
- **Startup hook.** Herdr runs `[[startup]]` after it restores the session and opens the socket. The
  hook calls `bin/run.sh reconcile`, so the usual binary and state paths apply.
- **Board on open.** A running Herdr can miss an event. The board runs the same prune before it
  reads the links.
- **Best effort.** A failed `pane list` leaves every link. A missing link is worse than a stale one
  at that moment.
- **The pure part is tested.** `prune_state` takes a state and a live set, so a test needs no Herdr
  and no state dir. `parse_live_panes` is tested with a captured pane list.

## Risks

- A pane created while the list runs is missing from the set, so its link could be pruned. The link
  is written after the pane exists, and the prune and the start are separate user actions, so the
  window is small.
- A linked pane that moves to another workspace gets a new id. Its old link is pruned; the start
  flow does not follow the move. That is correct: the link no longer points at a live task pane.
