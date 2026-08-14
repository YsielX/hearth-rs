return {
    api_version = 1,
    id = "LOOT_044",
    name = "Bladed Gauntlet",
    text = "Has Attack equal to your Armor. Can't attack heroes.",
    set = "LOOTAPALOOZA",
    type = "weapon",
    class = "warrior",
    rarity = "epic",
    cost = 2,
    attack = 0,
    health = 2,
    rules_inherit_to_hero = true,
    rules = {
        can_attack_character = function(ctx, self, current, target)
            return current and ctx:entity(target).type ~= "hero"
        end,
    },
    auras = {{
        active_zones = { "weapon" },
        attack = function(ctx, self)
            return ctx:entity(ctx:player(ctx:controller(self)).hero).armor
        end,
        targets = function(ctx, self) return { self } end,
    }},
}
