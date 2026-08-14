return {
    api_version = 1, id = "CFM_759", name = "Meanstreet Marshal",
    text = "<b>Deathrattle:</b> If this minion has 2 or more Attack, draw a card.",
    set = "GANGS", type = "minion", class = "paladin", rarity = "epic",
    cost = 1, attack = 1, health = 2, keywords = { "deathrattle" },
    on_deathrattle = function(ctx, self)
        local info = ctx:entity(self)
        if (info.attack_at_death or info.attack) >= 2 then ctx:draw(ctx:controller(self), 1) end
    end,
}
