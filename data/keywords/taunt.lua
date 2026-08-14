return {
    api_version = 1,
    module_type = "keyword",
    id = "taunt",
    name = "Taunt",
    rules = {
        attack_priority = function(ctx, self, current, attacker)
            return math.max(current, 1)
        end,
    },
}
