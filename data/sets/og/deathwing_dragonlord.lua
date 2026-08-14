local card = {
    api_version = 1, id = "OG_317", name = "Deathwing, Dragonlord",
    text = "<b>Deathrattle:</b> Put all Dragons from your hand into the battlefield.", set = "OG",
    type = "minion", rarity = "legendary", cost = 10, attack = 12, health = 12,
    tags = { "dragon" }, keywords = { "deathrattle" },
}

function card.on_deathrattle(ctx, self)
    local dragons = {}
    for _, entity in ipairs(ctx:hand(ctx:controller(self))) do
        local definition = ctx:card_definition(ctx:entity(entity).card_id)
        for _, tag in ipairs(definition.tags or {}) do
            if tag == "dragon" or tag == "all" then dragons[#dragons + 1] = entity break end
        end
    end
    for _, entity in ipairs(dragons) do ctx:summon_from_hand(entity) end
end

return card
