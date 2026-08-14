return {
    api_version = 1,
    module_type = "keyword",
    id = "hero_power_unlimited",
    name = "Hero Power Unlimited",
    rules = {
        max_uses_per_turn = function(ctx, self, current)
            return 255
        end,
    },
}
