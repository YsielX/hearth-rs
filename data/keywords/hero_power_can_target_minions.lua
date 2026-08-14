return {
    api_version = 1,
    module_type = "keyword",
    id = "hero_power_can_target_minions",
    name = "Hero Power Can Target Minions",
    rules = {
        requires_target = function(ctx, self, current)
            return true
        end,
    },
}
