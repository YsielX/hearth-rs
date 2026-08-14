local card = {
    api_version = 1,
    id = "VAC_437",
    name = "Buttons",
    text = "<b>Shaman Tourist</b>\n <b>Battlecry:</b> Draw a spell of each spell school.",
    set = "ISLAND_VACATION",
    type = "minion",
    class = "death_knight",
    rarity = "legendary",
    cost = 4,
    attack = 4,
    health = 4,
    tags = { "undead" },
    keywords = { "tourist", "battlecry" },
    deck_allowances = {
        {
            class = "shaman",
            set = "ISLAND_VACATION",
            excluded_keywords = { "tourist" },
        },
    },
}

function card.on_battlecry(ctx, self)
    local player = ctx:controller(self)
    local seen = {}
    local selected = {}
    for _, entity in ipairs(ctx:deck(player)) do
        local definition = ctx:card_definition(ctx:entity(entity).card_id)
        local school = definition.spell_school
        if definition.type == "spell" and school ~= nil and not seen[school] then
            seen[school] = true
            selected[#selected + 1] = entity
        end
    end

    -- Moving in reverse preserves the original deck order when each card is placed on top.
    for index = #selected, 1, -1 do
        ctx:move(selected[index], "deck_top")
    end
    if #selected > 0 then
        ctx:draw(player, #selected)
    end
end

return card
