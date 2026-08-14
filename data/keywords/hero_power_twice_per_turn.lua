return {
    api_version = 1,
    module_type = "keyword",
    id = "hero_power_twice_per_turn",
    name = "Hero Power Twice Per Turn",
    rules = {
        max_uses_per_turn = function(ctx, self, current)
            return math.max(current, 2)
        end,
    },
}
