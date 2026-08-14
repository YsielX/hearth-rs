local card = {
    api_version = 1, id = "LOE_029", name = "Jeweled Scarab",
    text = "<b>Battlecry: Discover</b> a\n3-Cost card.",
    set = "LOE", type = "minion", rarity = "common", cost = 2, attack = 1, health = 1,
    tags = { "beast" }, keywords = { "battlecry", "discover" },
}

function card.on_battlecry(ctx, self)
    local player = ctx:controller(self)
    local class = ctx:player(player).class
    local pool = {}
    for _, card_id in ipairs(ctx:collectible_cards()) do
        local definition = ctx:card_definition(card_id)
        if definition.cost == 3
            and (definition.class == "neutral" or definition.class == class) then
            pool[#pool + 1] = card_id
        end
    end
    ctx:discover_cards(player, "Discover a 3-Cost card", pool, 3, "receive_card")
end

function card.receive_card(ctx, self, card_id) ctx:give_card(ctx:controller(self), card_id) end
return card
