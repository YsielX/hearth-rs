local card = {
    api_version = 1,
    id = "KAR_057",
    name = "Ivory Knight",
    text = "[x]<b>Battlecry:</b> <b>Discover</b> a spell.\nRestore Health to your hero\nequal to its Cost.",
    set = "KARA",
    type = "minion",
    class = "paladin",
    rarity = "rare",
    cost = 4,
    attack = 4,
    health = 4,
    keywords = { "battlecry", "discover" },
}

function card.on_battlecry(ctx, self)
    local player = ctx:controller(self)
    local player_class = ctx:player(player).class
    local pool = {}
    for _, card_id in ipairs(ctx:collectible_cards()) do
        local definition = ctx:card_definition(card_id)
        if definition.type == "spell"
            and (definition.class == "neutral" or definition.class == player_class) then
            pool[#pool + 1] = card_id
        end
    end
    if #pool > 0 then
        ctx:discover_cards(player, "Discover a spell", pool, 3, "receive_spell")
    end
end

function card.receive_spell(ctx, self, card_id)
    local player = ctx:controller(self)
    ctx:give_card(player, card_id)
    ctx:heal(ctx:player(player).hero, ctx:card_definition(card_id).cost)
end

return card
