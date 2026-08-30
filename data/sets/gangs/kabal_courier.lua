local card = {
    api_version = 1, id = "CFM_649", name = "Kabal Courier",
    text = "<b>Battlecry:</b> <b>Discover</b>\na Mage, Priest, or\nWarlock card.",
    set = "GANGS", type = "minion", rarity = "rare", classes = { "mage", "priest", "warlock" },
    cost = 2, attack = 2, health = 2,
    keywords = { "battlecry" },
}
function card.on_battlecry(ctx, self)
    local pool = {}
    for _, id in ipairs(ctx:collectible_cards()) do
        local definition = ctx:card_definition(id)
        local eligible = definition.class == "mage" or definition.class == "priest" or definition.class == "warlock"
        for _, class in ipairs(definition.classes or {}) do
            if class == "mage" or class == "priest" or class == "warlock" then eligible = true end
        end
        if eligible then pool[#pool + 1] = id end
    end
    if #pool > 0 then ctx:discover_cards(ctx:controller(self), "Choose a Kabal card", pool, 3, "receive_kabal_card") end
end
function card.receive_kabal_card(ctx, self, id) cardlib.effects.give_card(ctx, ctx:controller(self), id) end
return card
