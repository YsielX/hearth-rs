local function is_dragon(definition)
    for _, tag in ipairs(definition.tags or {}) do
        if tag == "dragon" or tag == "all" then return true end
    end
    return false
end

local card = {
    api_version = 1, id = "ICC_027", name = "Bone Drake",
    text = "<b>Deathrattle:</b> Add a random Dragon to your hand.",
    set = "ICECROWN", type = "minion", rarity = "rare",
    cost = 6, attack = 6, health = 5, tags = { "undead", "dragon" }, keywords = { "deathrattle" },
}

function card.on_deathrattle(ctx, self)
    local pool = {}
    for _, card_id in ipairs(ctx:collectible_cards()) do
        local definition = ctx:card_definition(card_id)
        if definition.type == "minion" and is_dragon(definition) then pool[#pool + 1] = card_id end
    end
    if #pool > 0 then ctx:random_value(pool, "bone_drake_chosen") end
end

function card.bone_drake_chosen(ctx, self, card_id) cardlib.effects.give_card(ctx, ctx:controller(self), card_id) end

return card
