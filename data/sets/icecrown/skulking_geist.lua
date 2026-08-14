local card = {
    api_version = 1,
    id = "ICC_701",
    name = "Skulking Geist",
    text = "<b>Battlecry:</b> Destroy all\n1-Cost spells in both hands and decks.",
    set = "ICECROWN",
    type = "minion",
    rarity = "epic",
    cost = 6,
    attack = 4,
    health = 6,
    tags = { "undead" },
    keywords = { "battlecry" },
}

function card.on_battlecry(ctx, self)
    local owner = ctx:controller(self)
    for _, player in ipairs({ owner, ctx:opponent(owner) }) do
        for _, zone in ipairs({ ctx:hand(player), ctx:deck(player) }) do
            for _, entity in ipairs(zone) do
                local view = ctx:entity(entity)
                if view.type == "spell" and view.cost == 1 then ctx:move(entity, "graveyard") end
            end
        end
    end
end

return card
