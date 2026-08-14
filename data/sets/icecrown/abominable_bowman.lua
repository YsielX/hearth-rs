local function is_beast(definition)
    for _, tag in ipairs(definition.tags or {}) do
        if tag == "beast" or tag == "all" then return true end
    end
    return false
end

local card = {
    api_version = 1, id = "ICC_825", name = "Abominable Bowman",
    text = "[x]<b>Deathrattle:</b> Summon\na random friendly Beast\nthat died this game.",
    set = "ICECROWN", type = "minion", class = "hunter", rarity = "epic",
    cost = 7, attack = 6, health = 7, tags = { "undead" }, keywords = { "deathrattle" },
}

function card.on_deathrattle(ctx, self)
    local pool = {}
    for _, card_id in ipairs(ctx:minions_died(ctx:controller(self))) do
        if is_beast(ctx:card_definition(card_id)) then pool[#pool + 1] = card_id end
    end
    if #pool > 0 then ctx:random_value(pool, "abominable_bowman_chosen") end
end

function card.abominable_bowman_chosen(ctx, self, card_id)
    ctx:summon(ctx:controller(self), card_id)
end

return card
