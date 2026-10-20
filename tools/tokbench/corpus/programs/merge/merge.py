import json
from dataclasses import dataclass
from pathlib import Path


@dataclass
class Limits:
    memory_mb: int
    timeout_ms: int


@dataclass
class Config:
    name: str
    port: int
    debug: bool
    tags: list
    limits: Limits


def overlay(base, over):
    out = dict(base)
    for key, value in over.items():
        if isinstance(value, dict):
            value = overlay(base.get(key, {}), value)
        out[key] = value
    return out


def main():
    data = Path(__file__).resolve().parents[5] / "examples/configs"
    base = json.loads((data / "base.json").read_text())
    local = json.loads((data / "local.json").read_text())
    merged = overlay(base, local)
    cfg = Config(**{k: merged[k] for k in Config.__dataclass_fields__})
    cfg.limits = Limits(**cfg.limits)
    print(f"{cfg.name}:{cfg.port} debug={str(cfg.debug).lower()} tags={'+'.join(cfg.tags)}")
    print(f"limits {cfg.limits.memory_mb}MB {cfg.limits.timeout_ms}ms")
    for key in local:
        if key not in base:
            print(f"ignored {key}")


def test_overlays_nested_members():
    assert overlay({"a": {"x": 1, "y": 2}}, {"a": {"y": 3}}) == {"a": {"x": 1, "y": 3}}


if __name__ == "__main__":
    main()
