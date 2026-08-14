return {
    api_version = 1, id = "UNG_058", name = "Razorpetal Lasher",
    text = "[x]<b>Battlecry:</b> Add a\nRazorpetal to your hand\nthat deals 2 damage.",
    set = "UNGORO", type = "minion", class = "rogue", rarity = "common",
    cost = 2, attack = 2, health = 2, keywords = { "battlecry" },
    on_battlecry = function(ctx, self) ctx:give_card(ctx:controller(self), "UNG_057t1") end,
}
