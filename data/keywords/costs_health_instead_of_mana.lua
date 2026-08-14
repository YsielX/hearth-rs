return {
    api_version = 1,
    module_type = "keyword",
    id = "costs_health_instead_of_mana",
    name = "Costs Health Instead Of Mana",
    rules = {
        costs_health_instead_of_mana = function(ctx, self, current) return true end,
    },
}
