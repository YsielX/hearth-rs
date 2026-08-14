return {
    api_version = 1,
    id = "CFM_671",
    name = "Cryomancer",
    text = "<b>Battlecry:</b> If an enemy is <b>Frozen</b>, gain +2/+2.",
    set = "GANGS",
    type = "minion",
    class = "mage",
    rarity = "common",
    cost = 5,
    attack = 5,
    health = 5,
    keywords = { "battlecry" },
    on_battlecry = function(ctx, self)
        for _, enemy in ipairs(ctx:enemy_characters(self)) do
            if ctx:entity(enemy).frozen then
                ctx:buff(self, 2, 2)
                return
            end
        end
    end,
}
