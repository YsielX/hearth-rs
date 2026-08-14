return {
    api_version = 1, module_type = "keyword", id = "radiant_elemental_minimum_cost",
    name = "Radiant Elemental Minimum Cost",
    rules = { minimum_cost = function(ctx, self, current) return math.max(current, 1) end },
}
