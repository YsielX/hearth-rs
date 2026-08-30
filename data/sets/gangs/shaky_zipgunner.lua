local card = {
    api_version = 1,
    id = "CFM_336",
    name = "Shaky Zipgunner",
    text = "[x]<b>Deathrattle:</b> Give a random\nminion in your hand +2/+2.",
    set = "GANGS",
    type = "minion",
    class = "hunter",
    rarity = "common",
    cost = 3,
    attack = 4,
    health = 3,
    keywords = { "deathrattle" },
}

function card.on_deathrattle(ctx, self)
    local candidates = {}
    for _, entity in ipairs(ctx:hand(ctx:controller(self))) do
        if ctx:entity(entity).type == "minion" then candidates[#candidates + 1] = entity end
    end
    if #candidates > 0 then ctx:random_entity(candidates, "buff_hand_minion") end
end

function card.buff_hand_minion(ctx, self, target) cardlib.effects.buff(ctx, target, 2, 2) end

return card
