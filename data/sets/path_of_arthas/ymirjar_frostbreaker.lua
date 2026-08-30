local card = {
    api_version = 1,
    id = "RLK_110",
    name = "Ymirjar Frostbreaker",
    text = "<b>Battlecry:</b> Gain +1 Attack for each Frost spell\nin your hand.",
    set = "PATH_OF_ARTHAS",
    type = "minion",
    class = "death_knight",
    rarity = "common",
    cost = 1,
    attack = 1,
    health = 2,
    rune_cost = { frost = 1 },
    keywords = { "battlecry" },
}

function card.on_battlecry(ctx, self)
    local frost_spells = 0
    for _, entity in ipairs(ctx:hand(ctx:controller(self))) do
        local definition = ctx:card_definition(ctx:entity(entity).card_id)
        if definition.type == "spell" and definition.spell_school == "frost" then
            frost_spells = frost_spells + 1
        end
    end
    if frost_spells > 0 then cardlib.effects.buff(ctx, self, frost_spells, 0) end
end

return card
