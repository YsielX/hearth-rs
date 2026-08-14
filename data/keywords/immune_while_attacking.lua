return {
    api_version = 1, module_type = "keyword",
    id = "immune_while_attacking", name = "Immune While Attacking",
    weapon_inherits_to_hero = true,
    rules = {
        immune_while_attacking = function(ctx, self, current)
            return true
        end,
    },
}
