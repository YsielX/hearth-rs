return {
    api_version = 1,
    id = "CFM_065",
    name = "Volcanic Potion",
    text = "Deal $2 damage to all minions.",
    set = "GANGS",
    type = "spell",
    class = "mage",
    rarity = "rare",
    spell_school = "fire",
    cost = 3,
    on_play = function(ctx)
        local minions = ctx:minions()
        if #minions > 0 then cardlib.effects.damage_all(ctx, minions, 2) end
    end,
}
