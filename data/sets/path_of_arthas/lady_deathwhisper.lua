local card = {
    api_version = 1,
    id = "RLK_713",
    name = "Lady Deathwhisper",
    text = "<b>Deathrattle:</b> Copy all\nFrost spells in your hand.",
    set = "PATH_OF_ARTHAS",
    type = "minion",
    class = "death_knight",
    rarity = "legendary",
    cost = 4,
    attack = 4,
    health = 3,
    tags = { "undead" },
    rune_cost = { frost = 3 },
    keywords = { "deathrattle" },
}

function card.on_deathrattle(ctx, self)
    local player = ctx:controller(self)
    local frost_spells = {}
    for _, entity in ipairs(ctx:hand(player)) do
        local definition = ctx:card_definition(ctx:entity(entity).card_id)
        if definition.type == "spell" and definition.spell_school == "frost" then
            frost_spells[#frost_spells + 1] = entity
        end
    end
    for _, spell in ipairs(frost_spells) do ctx:give_copy(player, spell) end
end

return card
