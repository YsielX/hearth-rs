return {
    api_version = 1,
    module_type = "keyword",
    id = "rush",
    name = "Rush",
    rules = {
        can_attack_while_exhausted = function(ctx, self, current, defender)
            return current or (defender ~= nil and ctx:entity(defender).type == "minion")
        end,
    },
}
