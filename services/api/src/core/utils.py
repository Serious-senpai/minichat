from __future__ import annotations

from typing import Any, Callable, Coroutine, Generic, Optional, TypeVar, TYPE_CHECKING


__all__ = (
    "Singleton",
)
T = TypeVar("T")


class Singleton(Generic[T]):

    __slots__ = ("_instance", "_initializer")
    if TYPE_CHECKING:
        _instance: Optional[T]
        _initializer: Callable[[], Coroutine[Any, Any, T]]

    def __init__(self, _initializer: Callable[[], Coroutine[Any, Any, T]], /) -> None:
        self._instance = None
        self._initializer = _initializer

    async def __call__(self) -> T:
        if self._instance is None:
            self._instance = await self._initializer()

        return self._instance
