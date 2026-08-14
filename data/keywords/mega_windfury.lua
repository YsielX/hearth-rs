return {
    api_version = 1, module_type = "keyword", id = "mega_windfury", name = "Mega-Windfury",
    weapon_inherits_to_hero = true,
    rules = {
        max_attacks = function(ctx, self, current)
            return math.max(current, 4)
        end,
    },
}
