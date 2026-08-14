local function spell_pool(ctx)
    local result = {}
    for _, card_id in ipairs(ctx:collectible_cards()) do
        local definition = ctx:card_definition(card_id)
        local excluded = false
        for _, keyword in ipairs(definition.keywords or {}) do
            if keyword == "quest" or keyword == "questline" or keyword == "cannot_be_randomly_generated" then
                excluded = true
                break
            end
        end
        if definition.type == "spell" and not excluded then
            result[#result + 1] = card_id
        end
    end
    return result
end

return {
    api_version = 1,
    id = "OG_134",
    name = "Yogg-Saron, Hope's End",
    text = "[x]<b>Battlecry:</b> Cast a random\nspell for each spell you've\ncast this game <i>(targets\nchosen randomly)</i>.",
    set = "OG",
    type = "minion",
    rarity = "legendary",
    cost = 10,
    attack = 7,
    health = 5,
    keywords = { "battlecry" },
    on_battlecry = function(ctx, self)
        local player = ctx:controller(self)
        ctx:cast_random_spells(player, spell_pool(ctx), math.min(30, #ctx:spells_cast(player)))
    end,
}
