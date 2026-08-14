return {
    api_version = 1, id = "ICC_252", name = "Coldwraith",
    text = "<b>Battlecry:</b> If an enemy is <b>Frozen</b>, draw a card.",
    set = "ICECROWN", type = "minion", class = "mage", rarity = "common",
    cost = 3, attack = 3, health = 4, tags = { "undead" }, keywords = { "battlecry" },
    on_battlecry = function(ctx, self)
        for _, enemy in ipairs(ctx:enemy_characters(self)) do
            if ctx:entity(enemy).frozen then ctx:draw(ctx:controller(self), 1); return end
        end
    end,
}
