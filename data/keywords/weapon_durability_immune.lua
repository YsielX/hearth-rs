return {
    api_version = 1,
    module_type = "keyword",
    id = "weapon_durability_immune",
    name = "Weapon Durability Immune",
    rules = {
        durability_loss = function(ctx, self, current)
            return 0
        end,
    },
}
