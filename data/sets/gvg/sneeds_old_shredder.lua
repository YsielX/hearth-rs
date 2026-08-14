local card = {
    api_version = 1, id = "GVG_114", name = "Sneed's Old Shredder",
    text = "<b>Deathrattle:</b> Summon a random <b>Legendary</b> minion.", set = "GVG",
    type = "minion", rarity = "legendary", cost = 7, attack = 5, health = 7,
    tags = { "mech" }, keywords = { "deathrattle" },
}
function card.on_deathrattle(ctx, self)
    local pool = {}
    for _, id in ipairs(ctx:collectible_cards()) do
        local definition = ctx:card_definition(id)
        if definition.type == "minion" and definition.rarity == "legendary" then pool[#pool + 1] = id end
    end
    if #pool > 0 then ctx:random_value(pool, "summon_legendary") end
end
function card.summon_legendary(ctx, self, id) ctx:summon(ctx:controller(self), id) end
return card
