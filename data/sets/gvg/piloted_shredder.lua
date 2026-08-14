local card = {
    api_version = 1, id = "GVG_096", name = "Piloted Shredder",
    text = "<b>Deathrattle:</b> Summon a random 2-Cost minion.", set = "GVG",
    type = "minion", rarity = "common", cost = 4, attack = 4, health = 3,
    tags = { "mech" }, keywords = { "deathrattle" },
}
function card.on_deathrattle(ctx, self)
    local pool = {}
    for _, id in ipairs(ctx:collectible_cards()) do
        local definition = ctx:card_definition(id)
        if definition.type == "minion" and definition.cost == 2 then pool[#pool + 1] = id end
    end
    if #pool > 0 then ctx:random_value(pool, "summon_pilot") end
end
function card.summon_pilot(ctx, self, id) ctx:summon(ctx:controller(self), id) end
return card
