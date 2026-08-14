return {
    api_version = 1, module_type = "keyword",
    id = "weapon_cannot_attack_heroes", name = "Weapon Cannot Attack Heroes",
    weapon_inherits_to_hero = true,
    rules = {
        can_attack_character = function(ctx, self, current, target)
            return current and ctx:entity(target).type ~= "hero"
        end,
    },
}
