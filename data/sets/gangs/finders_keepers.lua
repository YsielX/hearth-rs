local card = {
    api_version = 1, id = "CFM_313", name = "Finders Keepers",
    text = "<b>Discover</b> a card with <b>Overload</b>. <b>Overload:</b> (1)",
    set = "GANGS", type = "spell", class = "shaman", rarity = "epic", cost = 1,
    keywords = { "discover", "overload" }, keyword_params = { overload = 1 },
}
local function has_overload(definition)
    for _, keyword in ipairs(definition.keywords or {}) do
        if keyword == "overload" then return true end
    end
    return false
end
function card.on_play(ctx, self)
    local pool = {}
    for _, card_id in ipairs(ctx:collectible_cards()) do
        if has_overload(ctx:card_definition(card_id)) then pool[#pool + 1] = card_id end
    end
    if #pool > 0 then
        ctx:discover_cards(ctx:controller(self), "Choose an Overload card", pool, 3, "receive_card")
    end
end
function card.receive_card(ctx, self, card_id) cardlib.effects.give_card(ctx, ctx:controller(self), card_id) end
return card
