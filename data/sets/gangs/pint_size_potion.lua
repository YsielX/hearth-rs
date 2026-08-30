return {
    api_version = 1, id = "CFM_661", name = "Pint-Size Potion",
    text = "[x]Give all enemy minions\n-3 Attack this turn only.",
    set = "GANGS", type = "spell", class = "priest", rarity = "rare", spell_school = "shadow", cost = 1,
    on_play = function(ctx, self)
        for _, entity in ipairs(ctx:enemy_characters(self)) do
            if ctx:entity(entity).type == "minion" then cardlib.effects.buff_until_end_of_turn(ctx, entity, -3, 0) end
        end
    end,
}
