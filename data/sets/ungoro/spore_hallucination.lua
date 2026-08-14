local card = {
    api_version = 1, id = "UNG_856", name = "Spore Hallucination",
    text = "<b>Discover</b> a card from your opponent's class.",
    set = "UNGORO", type = "spell", class = "rogue", rarity = "common", spell_school = "nature", cost = 1,
}
function card.on_play(ctx, self)
    local player = ctx:controller(self)
    local enemy_class = ctx:player(ctx:opponent(player)).class
    local pool = {}
    for _, id in ipairs(ctx:collectible_cards()) do
        local definition = ctx:card_definition(id)
        local eligible = definition.class == enemy_class
        for _, class in ipairs(definition.classes or {}) do if class == enemy_class then eligible = true end end
        if eligible then pool[#pool + 1] = id end
    end
    if #pool > 0 then ctx:discover_cards(player, "Choose a card", pool, 3, "receive_spore_card") end
end
function card.receive_spore_card(ctx, self, id) ctx:give_card(ctx:controller(self), id) end
return card
