return {
    api_version = 1, module_type = "keyword", id = "passive", name = "Passive",
    rules = {
        hero_power_is_passive = function(ctx, self, current) return true end,
    },
}
