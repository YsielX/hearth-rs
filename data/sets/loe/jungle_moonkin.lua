return {
    api_version = 1, id = "LOE_051", name = "Jungle Moonkin",
    text = "Both players have\n<b>Spell Damage +2</b>.",
    set = "LOE", type = "minion", class = "druid", rarity = "rare",
    cost = 4, attack = 4, health = 4,
    auras = {{
        spell_damage = 2,
        targets = function(ctx, self)
            local player = ctx:controller(self)
            return {
                ctx:player(player).hero,
                ctx:player(ctx:opponent(player)).hero,
            }
        end,
    }},
}
