local card = {
    api_version = 1,
    id = "LOOT_093",
    name = "Call to Arms",
    text = "[x]<b>Recruit</b> 3 minions that\n cost (2) or less.",
    set = "LOOTAPALOOZA",
    type = "spell",
    rarity = "epic",
    class = "paladin",
    cost = 4,
    keywords = { "recruit" },
    on_play = function(ctx, self)
        ctx:set_data(self, "call_to_arms_left", 3)
        ctx:continue_with("recruit_next_minion")
    end,
}

function card.recruit_next_minion(ctx, self)
    if ctx:get_data(self, "call_to_arms_left") <= 0
        or #ctx:board(ctx:controller(self)) >= 7 then return end
    local pool = {}
    for _, entity in ipairs(ctx:deck(ctx:controller(self))) do
        local view = ctx:entity(entity)
        if view.type == "minion" and view.cost <= 2 then pool[#pool + 1] = entity end
    end
    if #pool > 0 then ctx:random_entity(pool, "recruit_called_minion") end
end

function card.recruit_called_minion(ctx, self, entity)
    ctx:recruit(ctx:controller(self), entity)
    local left = ctx:get_data(self, "call_to_arms_left") - 1
    ctx:set_data(self, "call_to_arms_left", left)
    if left > 0 then ctx:continue_with("recruit_next_minion") end
end

return card
