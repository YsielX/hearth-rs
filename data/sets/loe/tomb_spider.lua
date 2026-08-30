local card = {
    api_version = 1,
    id = "LOE_047",
    name = "Tomb Spider",
    text = "<b>Battlecry: Discover</b> a Beast.",
    set = "LOE",
    type = "minion",
    rarity = "common",
    cost = 4,
    attack = 3,
    health = 3,
    tags = { "beast" },
    keywords = { "battlecry" },
}

function card.on_battlecry(ctx, self)
    local player = ctx:controller(self)
    local player_class = ctx:player(player).class
    local pool = {}
    for _, card_id in ipairs(ctx:collectible_cards()) do
        local definition = ctx:card_definition(card_id)
        if definition.type == "minion"
            and (definition.class == "neutral" or definition.class == player_class) then
            for _, tag in ipairs(definition.tags or {}) do
                if tag == "beast" or tag == "all" then
                    pool[#pool + 1] = card_id
                    break
                end
            end
        end
    end
    if #pool > 0 then ctx:discover_cards(player, "Choose a Beast", pool, 3, "receive_beast") end
end

function card.receive_beast(ctx, self, card_id)
    cardlib.effects.give_card(ctx, ctx:controller(self), card_id)
end

return card
