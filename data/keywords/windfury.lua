return {
    api_version = 1,
    module_type = "keyword",
    id = "windfury",
    name = "Windfury",
    weapon_inherits_to_hero = true,
    rules = {
        max_attacks = function(ctx, self, current, other)
            return math.max(current, 2)
        end,
    },
}
