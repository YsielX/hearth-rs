return {
    api_version = 1, id = "AT_059", name = "Brave Archer",
    text = "<b>Inspire:</b> If your hand is empty, deal 2 damage to the enemy hero.",
    set = "TGT", type = "minion", class = "hunter", rarity = "common",
    cost = 1, attack = 2, health = 1, keywords = { "inspire" },
    on_inspire = function(ctx, self)
        if #ctx:hand(ctx:controller(self)) == 0 then
            local opponent = ctx:opponent(ctx:controller(self))
            ctx:damage(ctx:player(opponent).hero, 2)
        end
    end,
}
