"""Example Starlark rule."""

def _impl(_ctx):
    return []

my_rule = rule(
    implementation = _impl,
)
