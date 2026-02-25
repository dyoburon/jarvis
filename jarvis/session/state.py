"""
jarvis/session/state.py

Panel state management for Jarvis.
Tracks active panels, focus, and skill tasks.

@module session/state
"""

import asyncio


# =============================================================================
# PANEL STATE
# =============================================================================


class PanelState:
    """
    Manages panel state for the Jarvis session.

    Tracks:
    - Active panel index
    - Panel count
    - Running skill tasks per panel
    - Pending tool name
    """

    def __init__(self, max_panels: int = 5):
        self.max_panels = max_panels
        self._active_panel: int = 0
        self._panel_count: int = 0
        self._skill_active: bool = False
        self._pending_tool_name: str | None = None
        self._skill_tasks: dict[int, asyncio.Task] = {}
        self._current_game: str | None = None

    # -------------------------------------------------------------------------
    # Properties
    # -------------------------------------------------------------------------

    @property
    def active_panel(self) -> int:
        """Get the currently active panel index."""
        return self._active_panel

    @active_panel.setter
    def active_panel(self, value: int) -> None:
        """Set the active panel index."""
        self._active_panel = max(0, min(value, max(0, self._panel_count - 1)))

    @property
    def panel_count(self) -> int:
        """Get the current number of panels."""
        return self._panel_count

    @panel_count.setter
    def panel_count(self, value: int) -> None:
        """Set the panel count."""
        self._panel_count = max(0, min(value, self.max_panels))

    @property
    def skill_active(self) -> bool:
        """Check if a skill is currently active."""
        return self._skill_active

    @skill_active.setter
    def skill_active(self, value: bool) -> None:
        """Set skill active state."""
        self._skill_active = value

    @property
    def pending_tool_name(self) -> str | None:
        """Get the pending tool name."""
        return self._pending_tool_name

    @pending_tool_name.setter
    def pending_tool_name(self, value: str | None) -> None:
        """Set the pending tool name."""
        self._pending_tool_name = value

    @property
    def current_game(self) -> str | None:
        """Get the currently active game."""
        return self._current_game

    @current_game.setter
    def current_game(self, value: str | None) -> None:
        """Set the currently active game."""
        self._current_game = value

    # -------------------------------------------------------------------------
    # Panel Operations
    # -------------------------------------------------------------------------

    def add_panel(self) -> int:
        """
        Add a new panel.

        Returns:
            The new panel index, or -1 if max panels reached.
        """
        if self._panel_count >= self.max_panels:
            return -1

        new_panel = self._panel_count
        self._panel_count += 1
        self._active_panel = new_panel
        return new_panel

    def remove_panel(self, index: int) -> bool:
        """
        Remove a panel at the given index.

        Args:
            index: The panel index to remove.

        Returns:
            True if panel was removed, False if invalid index.
        """
        if index < 0 or index >= self._panel_count:
            return False

        if self._panel_count <= 1:
            # Can't remove last panel - this should close everything
            return False

        # Cancel any task on this panel
        task = self._skill_tasks.pop(index, None)
        if task and not task.done():
            task.cancel()

        # Renumber tasks for panels above the removed one
        new_tasks: dict[int, asyncio.Task] = {}
        for pid, t in self._skill_tasks.items():
            new_tasks[pid - 1 if pid > index else pid] = t
        self._skill_tasks = new_tasks

        self._panel_count -= 1
        self._active_panel = min(index, self._panel_count - 1)

        return True

    def close_all_panels(self) -> None:
        """Close all panels and cancel all tasks."""
        for task in self._skill_tasks.values():
            if not task.done():
                task.cancel()

        self._skill_tasks.clear()
        self._panel_count = 0
        self._active_panel = 0
        self._skill_active = False
        self._pending_tool_name = None

    # -------------------------------------------------------------------------
    # Task Management
    # -------------------------------------------------------------------------

    def set_task(self, panel: int, task: asyncio.Task) -> None:
        """Set the task for a panel."""
        self._skill_tasks[panel] = task

    def get_task(self, panel: int) -> asyncio.Task | None:
        """Get the task for a panel."""
        return self._skill_tasks.get(panel)

    def has_running_task(self, panel: int) -> bool:
        """Check if a panel has a running (not done) task."""
        task = self._skill_tasks.get(panel)
        return task is not None and not task.done()

    def cancel_task(self, panel: int) -> bool:
        """
        Cancel the task for a panel.

        Returns:
            True if a task was cancelled, False otherwise.
        """
        task = self._skill_tasks.pop(panel, None)
        if task and not task.done():
            task.cancel()
            return True
        return False

    def cancel_all_tasks(self) -> None:
        """Cancel all running tasks."""
        for task in self._skill_tasks.values():
            if not task.done():
                task.cancel()
        self._skill_tasks.clear()

    # -------------------------------------------------------------------------
    # State Helpers
    # -------------------------------------------------------------------------

    def can_spawn_panel(self) -> bool:
        """Check if a new panel can be spawned."""
        return self._panel_count > 0 and self._panel_count < self.max_panels

    def is_last_panel(self, index: int) -> bool:
        """Check if the given index is the last/only panel."""
        return self._panel_count == 1 and index == 0

    def panel_name(self, index: int) -> str:
        """Get the display name for a panel."""
        return f"Opus 4.6 Assistant {index + 1}"

    def reset(self) -> None:
        """Reset all state to initial values."""
        self.cancel_all_tasks()
        self._panel_count = 0
        self._active_panel = 0
        self._skill_active = False
        self._pending_tool_name = None
        self._current_game = None
