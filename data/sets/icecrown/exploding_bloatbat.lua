return {
    api_version = 1, id = "ICC_021", name = "Exploding Bloatbat",
    text = "<b>Deathrattle:</b>\nDeal 2 damage to all enemy minions.",
    set = "ICECROWN", type = "minion", class = "hunter", rarity = "rare",
    cost = 4, attack = 2, health = 1, tags = { "beast" }, keywords = { "deathrattle" },
    on_deathrattle = function(ctx, self)
        local targets = {}
        local opponent = ctx:opponent(ctx:controller(self))
        for _, minion in ipairs(ctx:minions()) do
            if ctx:controller(minion) == opponent then targets[#targets + 1] = minion end
        end
        ctx:damage_all(targets, 2)
    end,
}
