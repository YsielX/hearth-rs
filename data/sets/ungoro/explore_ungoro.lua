local card = {
    api_version = 1, id = "UNG_922", name = "Explore Un'Goro",
    text = "Replace your deck with copies of \"<b>Discover</b> a card.\"",
    set = "UNGORO", type = "spell", class = "warrior", rarity = "epic", cost = 2,
}
function card.on_play(ctx, self)
    for _, entity in ipairs(ctx:deck(ctx:controller(self))) do ctx:transform(entity, "UNG_922t1") end
end
local choice = { id = "UNG_922t1", name = "Choose Your Path", text = "<b>Discover</b> a card.", set = "UNGORO", type = "spell", class = "warrior", collectible = false, cost = 0, keywords = { "discover" } }
function choice.on_play(ctx, self)
    local pool, player = {}, ctx:controller(self)
    for _, id in ipairs(ctx:collectible_cards()) do
        local definition = ctx:card_definition(id)
        local allowed = definition.class == "neutral" or definition.class == ctx:player(player).class
        for _, class in ipairs(definition.classes or {}) do if class == ctx:player(player).class then allowed = true end end
        if allowed then pool[#pool + 1] = id end
    end
    if #pool > 0 then ctx:discover_cards(player, "Discover a card", pool, 3, "receive_discovered_card") end
end
function choice.receive_discovered_card(ctx, self, id) ctx:give_card(ctx:controller(self), id) end
card.tokens = { choice }
return card
