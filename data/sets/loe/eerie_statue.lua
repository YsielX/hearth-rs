return {
    api_version = 1,
    id = "LOE_107",
    name = "Eerie Statue",
    text = "Can’t attack unless it’s the only minion on the battlefield.",
    set = "LOE",
    type = "minion",
    rarity = "rare",
    cost = 4,
    attack = 7,
    health = 7,
    rules = {
        can_attack = function(ctx, self, current)
            return current and #ctx:minions() == 1
        end,
    },
}
